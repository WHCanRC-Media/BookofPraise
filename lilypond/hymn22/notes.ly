melody = \relative c' {
  \clef treble
  \key f \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  c'2 d4 c4 bes2 a2 g2 r2 \break

  % Line 2
  bes2 a4 f4 g4 g4 f2 r2 \break

  % Line 3
  f2 c'4 c4 d4 c4 bes2 a2 r2 \break

  % Line 4
  c2 a4 bes4 a4 g4 f2 r2 \break

  % Line 5
  f2 d4 e4 f4 d4 c2 r2 \break

  % Line 6
  f2 g4 a4 bes4 a4 g2 f1 \bar "|."
}
