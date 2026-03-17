melody = \relative c' {
  \clef treble
  \key f \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  g'2 g2 f4 g4 a4 bes4 a2 g2 r2 \break

  % Line 2
  g2 g2 f4 g4 a4 bes4 a2 g2 r2 \break

  % Line 3
  bes2 a2 g4 bes4 a4 g4 f2 r2 \break

  % Line 4
  f2 e2 d4 g4 g4 fis4 g2 r2 \break

  % Line 5
  bes2 c2 d4 d4 c4 bes4 c2 bes2 r2 \break

  % Line 6
  bes2 c2 d4 d4 c4 bes4 c2 bes2 r2 \break

  % Line 7
  bes2 a2 g4 bes4 a4 g4 f2 r2 \break

  % Line 8
  f2 e2 d4 g4 g4 fis4 g1 \bar "|."
}
