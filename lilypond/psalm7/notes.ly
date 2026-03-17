melody = \relative c' {
  \clef treble
  \key f \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  r8 g'2 g4 a4 bes4 g4 c4 bes4 a2 g2 r2 \break

  % Line 2
  g2 g4 f4 g4 d4 f4 f4 e2 d2 r2 \break

  % Line 3
  d2 f4 f4 g4 a4 bes4 bes4 a2 r2 \break

  % Line 4
  f2 g4 bes4 a4 g4 g4 fis4 g2 \break

  % Line 5
  g2 bes4 bes4 a4 d4 c4 bes4 c2 bes2 \break

  % Line 6
  d2 c4 bes4 a4 f4 g4 f4 e2 d2 r2 \break

  % Line 7
  g2 g4 f4 e4 d4 f2 g2 a2 r2 \break

  % Line 8
  d,2 f4 e4 d4 g4 g4 fis4 \bar "|."
}
