melody = \relative c' {
  \clef treble
  \key f \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  f2 f4 g4 a4 g2 f2 e4 f2 r2 \break

  % Line 2
  a2 a4 g4 a2 c2 bes4 a4 g2 r2 \break

  % Line 3
  a2 c4 c4 d4 c4 bes4 a4 g2 f2 r2 \break

  % Line 4
  c2 d4 e4 f4 a4 g4 f4 e2 f2 r2 \break

  % Line 5
  f2 g2 a2 f4 g4 a4 bes4 c2 r2 \break

  % Line 6
  f,2 g2 a2 f4 e4 d4 d4 c2 r2 \break

  % Line 7
  c'2 c4 a4 f2 bes2 a4 g4 a2 r2 \break

  % Line 8
  a2 g4 f4 e2 g2 f4 e4 f1 \bar "|."
}
